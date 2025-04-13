package dev.upcraft.winterjam.util

import dev.kordex.core.utils.loadModule
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy
import java.util.*
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid
import kotlin.uuid.toJavaUuid

@OptIn(ExperimentalUuidApi::class)
class PlayerDbService(private val client: HttpClient) {

	@Throws(Exception::class)
	suspend fun getPlayerInfo(usernameOrUuid: String): PlayerInfoResponse {
		val url = "https://playerdb.co/api/player/minecraft/${usernameOrUuid}"

		val response = client.get(url) {
			accept(ContentType.Application.Json)
		}

		if(!response.status.isSuccess()) {
			throw Exception("Failed to get player info from PlayerDB: ${response.bodyAsText()}")
		}

		val body = response.body<PlayerDbResponse>()
		if(body.error == true) {
			throw Exception("PlayerDB server returned error: ${body.message}")
		}

		if(!body.success) {
			throw Exception("PlayerDB returned unsuccessful response: ${body.message}")
		}

		return body.data?.player?.let {
			PlayerInfoResponse(it.username, it.id.toJavaUuid(), it.avatar)
		} ?: throw Exception("Response body did not contain data: $body")
	}

	companion object {
		fun init() {
			@OptIn(ExperimentalSerializationApi::class)
			val client = HttpClient {
				install(ContentNegotiation) {
					json(Json {
						ignoreUnknownKeys = true
						namingStrategy = JsonNamingStrategy.SnakeCase
					})
				}
			}

			loadModule {
				single { PlayerDbService(client) }
			}
		}
	}
}

data class PlayerInfoResponse(
	val name: String,
	val uuid: UUID,
	val avatarUrl: String?,
)

@Serializable
data class PlayerDbResponse(
	val code: String,
	val message: String? = null,
	val data: Data? = null,
	val success: Boolean,
	val error: Boolean? = null,
) {
	@Serializable
	data class Data(
		val player: Player? = null,
	) {

		@OptIn(ExperimentalUuidApi::class)
		@Serializable
		data class Player(
			val meta: Meta,
			val username: String,
			val id: Uuid,
			val rawId: String,
			val avatar: String? = null,
			val skinTexture: String? = null,
			val properties: List<ProfileProperty>? = null,
		) {
			@Serializable
			data class Meta(
				val cachedAt: Long,
			)

			@Serializable
			data class ProfileProperty(
				val name: String,
				val value: String,
				val signature: String? = null,
			)
		}
	}
}
