package dev.upcraft.winterjam.extensions.whitelist

import dev.kord.common.entity.Snowflake
import dev.kordex.core.extensions.Extension
import dev.kordex.modules.web.core.backend.errors.WebError
import dev.kordex.modules.web.core.backend.utils.apiRoutes
import dev.upcraft.winterjam.model.WhitelistEntryRepository
import dev.upcraft.winterjam.util.NotFound
import dev.upcraft.winterjam.util.respondError
import io.ktor.http.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable
import org.koin.core.component.inject
import java.util.*
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class)
@Serializable
data class UserInfo(
	val uuid: Uuid,
	val snowflake: Snowflake,
	val operator: Boolean?,
	val access: Boolean
)

fun addWhitelistApiRoutes(extension: Extension) {
	with(extension) {

		val whitelistEntries by inject<WhitelistEntryRepository>()

		apiRoutes {
			get("/{guild}") {
				val guild : Snowflake
				try {
					guild = Snowflake(call.pathParameters["guild"]!!)
				} catch (e: Exception) {
					return@get call.respondError(WebError(e.message ?: "Invalid guild ID", HttpStatusCode.BadRequest))
				}

				whitelistEntries.getWhitelistForGuild(guild)?.let { call.respond(it) } ?: call.respondError(NotFound)
			}

			get("/{guild}/{uuid}") {
				val guild : Snowflake
				try {
					guild = Snowflake(call.pathParameters["guild"]!!)
				} catch (e: Exception) {
					return@get call.respondError(WebError(e.message ?: "Invalid guild ID: ${call.pathParameters["guild"]}", HttpStatusCode.BadRequest))
				}

				val uuid: UUID
				try {
					uuid = UUID.fromString(call.pathParameters["uuid"]!!)
				} catch (e: IllegalArgumentException) {
					return@get call.respondError(WebError(e.message ?: "Invalid UUID: ${call.pathParameters["uuid"]}", HttpStatusCode.BadRequest))
				}

				whitelistEntries.getUserWhitelist(guild, uuid)?.let { call.respond(it) } ?: call.respondError(NotFound)
			}
		}
	}
}
