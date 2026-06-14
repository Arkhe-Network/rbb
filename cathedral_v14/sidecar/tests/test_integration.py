import asyncio
import uuid
import pytest
from aiohttp import web
from aiohttp.test_utils import AioHTTPTestCase, unittest_run_loop
from unittest.mock import AsyncMock, patch

from sidecar.client import GgufSidecarClient

class TestCircuitBreaker(AioHTTPTestCase):
    async def get_application(self):
        async def hello(request):
            return web.json_response({"text": "Hello, world!", "tokens": 4, "cache_hit": False})

        async def auth_fail(request):
            return web.Response(status=401, text="Unauthorized")

        async def server_error(request):
            return web.Response(status=500, text="Internal Server Error")

        async def timeout_simulate(request):
            await asyncio.sleep(0.5)
            return web.json_response({"text": "Slow response", "tokens": 2, "cache_hit": False})

        app = web.Application()
        app.router.add_post('/v1/generate', hello)
        app.router.add_post('/v1/generate/auth_fail', auth_fail)
        app.router.add_post('/v1/generate/server_error', server_error)
        app.router.add_post('/v1/generate/timeout', timeout_simulate)

        return app

    @unittest_run_loop
    async def test_successful_request(self):
        client = GgufSidecarClient(config={
            "sidecar_url": f"http://{self.server.host}:{self.server.port}",
            "sidecar_token": "cathedral-super-secret-token"
        })

        result = await client.generate("test prompt")
        self.assertEqual(result["text"], "Hello, world!")
        self.assertTrue(client.circuit.is_available())

        await client.close()
