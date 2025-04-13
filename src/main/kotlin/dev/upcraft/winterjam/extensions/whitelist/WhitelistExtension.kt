package dev.upcraft.winterjam.extensions.whitelist

import dev.kord.common.Color
import dev.kord.common.entity.Snowflake
import dev.kord.core.behavior.channel.createMessage
import dev.kord.core.behavior.getChannelOfOrNull
import dev.kord.core.entity.channel.GuildMessageChannel
import dev.kord.core.event.guild.BanAddEvent
import dev.kord.core.event.guild.MemberLeaveEvent
import dev.kord.rest.builder.message.embed
import dev.kordex.core.checks.anyGuild
import dev.kordex.core.checks.isNotBot
import dev.kordex.core.commands.Arguments
import dev.kordex.core.commands.application.slash.ephemeralSubCommand
import dev.kordex.core.commands.converters.impl.string
import dev.kordex.core.extensions.Extension
import dev.kordex.core.extensions.ephemeralSlashCommand
import dev.kordex.core.extensions.event
import dev.kordex.core.i18n.withContext
import dev.kordex.core.utils.envOf
import dev.kordex.core.utils.scheduling.Scheduler
import dev.upcraft.winterjam.i18n.Translations
import dev.upcraft.winterjam.model.DiscordUserRepository
import dev.upcraft.winterjam.model.MinecraftUserRepository
import dev.upcraft.winterjam.model.WhitelistEntryRepository
import dev.upcraft.winterjam.util.PlayerDbService
import kotlinx.datetime.Clock
import org.jetbrains.exposed.sql.transactions.transaction
import org.koin.core.component.inject
import kotlin.time.Duration.Companion.hours

class WhitelistExtension : Extension() {

	companion object {
		const val EXTENSION_NAME = "whitelist"
	}

	override val name = EXTENSION_NAME

	private val discordUsers by inject<DiscordUserRepository>()
	private val minecraftUsers by inject<MinecraftUserRepository>()
	private val whitelistEntries by inject<WhitelistEntryRepository>()
	private val playerDbService by inject<PlayerDbService>()

	private val scheduler = Scheduler()

	private val notificationChannelId: Snowflake = envOf<Snowflake>("WHITELIST_NOTIFICATION_CHANNEL")

	override suspend fun setup() {
		PlayerDbService.init()

		ephemeralSlashCommand {
			name = Translations.Commands.Whitelist.name
			description = Translations.Commands.Whitelist.description

			check {
				anyGuild()
			}

			ephemeralSubCommand(::WhitelistAddArgs) {
				name = Translations.Commands.Whitelist.Add.name
				description = Translations.Commands.Whitelist.Add.description

				action {
					val user = event.interaction.user

					val discordUser = discordUsers.getOrCreateUser(user, guild!!)
					val discordGuild = discordUsers.getGuild(guild!!)

					val playerInfo = playerDbService.getPlayerInfo(arguments.usernameOrUuid)

					val minecraftUser = minecraftUsers.getOrCreateUser(playerInfo.uuid, playerInfo.name)
					val minecraftUuid = minecraftUser.id.toString()
					val minecraftUsername = minecraftUser.displayName ?: minecraftUuid

					if(!whitelistEntries.createEntry(discordUser, guild!!.id, minecraftUser)) {
						respond {
							content = Translations.Commands.Whitelist.Add.Response.alreadyWhitelisted
								.withContext(this@action)
								.translateNamed(
									"user" to user.mention,
									"minecraft_username" to minecraftUsername,
									"minecraft_uuid" to minecraftUuid,
								)
						}

						return@action
					}

					guild!!.getChannelOfOrNull<GuildMessageChannel>(notificationChannelId)?.apply {
						createMessage {
							embed {
								title = Translations.Commands.Whitelist.Add.Embed.title
									.withContext(this@action)
									.translateNamed(
										"user" to user.mention,
										"minecraft_username" to minecraftUsername,
										"minecraft_uuid" to minecraftUuid,
									)
								description = Translations.Commands.Whitelist.Add.Embed.text
									.withContext(this@action)
									.translateNamed(
										"user" to user.mention,
										"minecraft_username" to minecraftUsername,
										"minecraft_uuid" to minecraftUuid,
									)
								footer {
									text = Translations.Commands.Whitelist.Add.Embed.footer
										.withContext(this@action)
										.translateNamed(
											"user" to user.mention,
											"minecraft_username" to minecraftUsername,
											"minecraft_uuid" to minecraftUuid,
										)
								}
								playerInfo.avatarUrl?.let {
									thumbnail {
										url = it
									}
								}
								color = Color(0x32a852)
								timestamp = Clock.System.now()
							}
						}
					}

					respond {
						content = Translations.Commands.Whitelist.Add.Response.success
							.withContext(this@action)
							.translateNamed(
								"user" to user.mention,
								"minecraft_username" to minecraftUsername,
								"minecraft_uuid" to minecraftUuid,
							)
					}
				}
			}

			// TODO remove subcommand (remove own entries only)
			// TODO remove subcommand (moderators)
			// TODO operator subcommands?

			addWhitelistApiRoutes(this@WhitelistExtension)
		}

		scheduler.schedule(3.hours, true, "whitelist sync", 30, true) {
			for (dbGuild in discordUsers.allGuilds(true)) {
				kord.getGuildOrNull(Snowflake(dbGuild.id.value))?.also { guild ->
					dbGuild.forEachUser { dbUser ->
						if(guild.getMemberOrNull(Snowflake(dbUser.id.value)) == null) {
							transaction {
								dbUser.leaveGuild(dbGuild)
							}
						}
					}
				} ?: discordUsers.deleteGuild(dbGuild)
			}
		}

		event<MemberLeaveEvent> {
			check {
				isNotBot()
			}

			action {
				discordUsers.getUser(event.user.id)?.leaveGuild(event.guildId)
			}
		}

		event<BanAddEvent> {
			check {
				isNotBot()
			}

			action {
				discordUsers.getUser(event.user.id)?.leaveGuild(event.guildId)
			}
		}
	}

	inner class WhitelistAddArgs : Arguments() {
		val usernameOrUuid by string {
			name = Translations.Arguments.MinecraftUser.name
			description = Translations.Arguments.MinecraftUser.description
		}
	}
}
