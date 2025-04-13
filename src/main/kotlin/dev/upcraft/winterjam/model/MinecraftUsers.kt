package dev.upcraft.winterjam.model

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.datetime.Clock
import org.jetbrains.exposed.dao.UUIDEntity
import org.jetbrains.exposed.dao.UUIDEntityClass
import org.jetbrains.exposed.dao.id.EntityID
import org.jetbrains.exposed.dao.id.UUIDTable
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.SchemaUtils
import org.jetbrains.exposed.sql.SizedIterable
import org.jetbrains.exposed.sql.kotlin.datetime.CurrentTimestamp
import org.jetbrains.exposed.sql.kotlin.datetime.timestamp
import org.jetbrains.exposed.sql.transactions.transaction
import java.util.*

object MinecraftUsers: UUIDTable("minecraft_users", "uuid") {

	val displayName = varchar("display_name", 16).nullable()

	val bannedAt = timestamp("banned_at").nullable()

	val createdAt = timestamp("created_at").defaultExpression(CurrentTimestamp)
	val updatedAt = timestamp("updated_at").defaultExpression(CurrentTimestamp)

	init {
		transaction {
			SchemaUtils.create(MinecraftUsers)
		}
	}
}

class MinecraftUser(uuid: EntityID<UUID>) : UUIDEntity(uuid) {
	companion object : UUIDEntityClass<MinecraftUser>(MinecraftUsers)

	var displayName by MinecraftUsers.displayName
	var bannedAt by MinecraftUsers.bannedAt

	var createdAt by MinecraftUsers.createdAt
	var updatedAt by MinecraftUsers.updatedAt

	fun isBanned(): Boolean = bannedAt != null

	fun update(block: MinecraftUser.() -> Unit) {
		transaction {
			val old = updatedAt
			apply(block)
			if(updatedAt == old) {
				updatedAt = Clock.System.now()
			}
		}
	}
}

class MinecraftUserRepository(private val database: Database) {

	suspend fun getOrCreateUser(uuid: UUID, displayName: String?): MinecraftUser = withContext(
		Dispatchers.IO) {
		transaction(database) {
			val existingUser = MinecraftUser.findById(uuid)

			existingUser?.also { user ->
				val needsUpdate = displayName != null && user.displayName != displayName

				if(needsUpdate) {
					user.update {
						this.displayName = displayName
					}
				}
			} ?: MinecraftUser.new(uuid) {
				this.displayName = displayName
			}
		}
	}

	suspend fun getUser(uuid: UUID): MinecraftUser? = withContext(Dispatchers.IO) {
		transaction(database) {
			MinecraftUser.findById(uuid)
		}
	}

	suspend fun getUsers(uuids: List<UUID>): SizedIterable<MinecraftUser> = withContext(Dispatchers.IO) {
		transaction(database) {
			MinecraftUser.find {
				MinecraftUsers.id inList uuids
			}
		}
	}

	suspend fun deleteUser(uuid: UUID) = withContext(Dispatchers.IO) {
		transaction(database) {
			MinecraftUser.findById(uuid)?.delete()
		}
	}

	suspend fun count(): Long = withContext(Dispatchers.IO) {
		transaction(database) {
			MinecraftUser.count()
		}
	}
}
