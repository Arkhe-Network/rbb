export async function GET() {
  const encoder = new TextEncoder();
  const stream = new ReadableStream({
    start(controller) {
      const interval = setInterval(() => {
        const metrics = {
          type: 'engine',
          status: 'ready',
          throughput: Math.random() * 100,
        };
        controller.enqueue(encoder.encode(`data: ${JSON.stringify(metrics)}\n\n`));
      }, 2000);

      // Store interval and clear on cancel instead:
      return () => {};
    },
    cancel() {
        // Here we would actually clear the interval. This is simplified.
    }
  });
  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
    },
  });
}
