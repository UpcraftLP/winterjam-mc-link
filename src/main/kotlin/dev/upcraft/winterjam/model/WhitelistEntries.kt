package dev.upcraft.winterjam.model

import dev.kord.common.entity.Snowflake
import dev.upcraft.winterjam.extensions.whitelist.UserInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.datetime.Clock
import org.jetbrains.exposed.dao.CompositeEntity
import org.jetbrains.exposed.dao.CompositeEntityClass
import org.jetbrains.exposed.dao.id.CompositeID
import org.jetbrains.exposed.dao.id.CompositeIdTable
import org.jetbrains.exposed.dao.id.EntityID
import org.jetbrains.exposed.dao.load
import org.jetbrains.exposed.dao.with
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.ReferenceOption
import org.jetbrains.exposed.sql.SchemaUtils
import org.jetbrains.exposed.sql.SqlExpressionBuilder.eq
import org.jetbrains.exposed.sql.and
import org.jetbrains.exposed.sql.kotlin.datetime.CurrentTimestamp
import org.jetbrains.exposed.sql.kotlin.datetime.timestamp
import org.jetbrains.exposed.sql.transactions.transaction
import java.util.*
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.toKotlinUuid

const val tableName = "minecraft_whitelist_entries"

object WhitelistEntries : CompositeIdTable(tableName) {

	val discordUserId = reference("discord_user_id", DiscordUsers, onUpdate = ReferenceOption.CASCADE, onDelete = ReferenceOption.CASCADE)
	val discordGuildId = reference("discord_guild_id", DiscordGuilds, onUpdate = ReferenceOption.CASCADE, onDelete = ReferenceOption.CASCADE)
	val minecraftUserId = reference("minecraft_user_id", MinecraftUsers, onUpdate = ReferenceOption.CASCADE, onDelete = ReferenceOption.CASCADE)

	val createdAt = timestamp("created_at").defaultExpression(CurrentTimestamp)
	val updatedAt = timestamp("updated_at").defaultExpression(CurrentTimestamp)

	override val primaryKey = PrimaryKey(discordUserId, discordGuildId, minecraftUserId, name = "PK_${tableName}")

	init {
		addIdColumn(discordUserId)
		addIdColumn(discordGuildId)
		addIdColumn(minecraftUserId)
		SchemaUtils.create(WhitelistEntries)

		index("idx_${tableName}_by_discord_user_id", false, discordUserId)
		index("idx_${tableName}_by_discord_guild_id", false, discordGuildId)
		index("idx_${tableName}_by_minecraft_user_id", false, minecraftUserId)
	}
}

class WhitelistEntry(id: EntityID<CompositeID>) : CompositeEntity(id) {
	companion object : CompositeEntityClass<WhitelistEntry>(WhitelistEntries)

	val discordUser by DiscordUser referencedOn WhitelistEntries.discordUserId
	val discordGuild by DiscordGuild referencedOn WhitelistEntries.discordGuildId
	val minecraftUser by MinecraftUser referencedOn WhitelistEntries.minecraftUserId

	var createdAt by WhitelistEntries.createdAt
	var updatedAt by WhitelistEntries.updatedAt

	fun update(block: WhitelistEntry.() -> Unit) {
		transaction {
			val old = updatedAt
			apply(block)
			if (updatedAt == old) {
				updatedAt = Clock.System.now()
			}
		}
	}
}

class WhitelistEntryRepository(private val database: Database) {

	suspend fun createEntry(
		discordUser: DiscordUser,
		guildId: Snowflake,
		minecraftUser: MinecraftUser
	): Boolean =
		withContext(Dispatchers.IO) {
			transaction(database) {
				val whitelistId = CompositeID {
					it[WhitelistEntries.discordUserId] = discordUser.id
					it[WhitelistEntries.discordGuildId] = guildId.value
					it[WhitelistEntries.minecraftUserId] = minecraftUser.id
				}

				if (WhitelistEntry.findById(whitelistId) != null) {
					return@transaction false
				}

				WhitelistEntry.new(whitelistId) {}

				return@transaction true
			}
		}

	@OptIn(ExperimentalUuidApi::class)
	suspend fun getUserWhitelist(guildId: Snowflake, uuid: UUID): UserInfo? = withContext(Dispatchers.IO) {
		transaction(database) {
			WhitelistEntry.find {
				(WhitelistEntries.discordGuildId eq guildId.value) and (WhitelistEntries.minecraftUserId eq uuid)
			}
				.firstOrNull()?.load(WhitelistEntry::discordUser, WhitelistEntry::minecraftUser)?.let {
					UserInfo(
						uuid.toKotlinUuid(),
						Snowflake(it.discordUser.id.value),
						null,
						access = !it.minecraftUser.isBanned()
					)
				}
		}
	}

	@OptIn(ExperimentalUuidApi::class)
	suspend fun getWhitelistForGuild(guildId: Snowflake): List<UserInfo>? = withContext(Dispatchers.IO) {
		transaction(database) {
			if(DiscordGuild.findById(guildId.value) == null) {
				return@transaction null
			}

			WhitelistEntry.find { WhitelistEntries.discordGuildId eq guildId.value }
				.with(WhitelistEntry::discordUser, WhitelistEntry::minecraftUser)
				.map { UserInfo(
					it.minecraftUser.id.value.toKotlinUuid(),
					Snowflake(it.discordUser.id.value),
					null,
					access = !it.minecraftUser.isBanned()
				) }
		}
	}
}

enum class WhitelistState {
	ALLOWED,
	BANNED,
}
