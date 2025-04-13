package dev.upcraft.winterjam.util

import dev.kordex.modules.web.core.backend.errors.WebError
import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*

public val NotFound: WebError = WebError("Not found", HttpStatusCode.NotFound)

public suspend fun ApplicationCall.respondError(error: WebError) {
	this.respond(error.statusCode, error.message)
}
