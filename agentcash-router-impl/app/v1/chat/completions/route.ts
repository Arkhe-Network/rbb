import { z } from 'zod';
import { router } from '../../../../lib/router';

const ChatCompletionSchema = z.object({
  model: z.string().optional().default("Rio-3.5-Open-397B"),
  messages: z.array(
    z.object({
      role: z.enum(['system', 'user', 'assistant']),
      content: z.string(),
    })
  ).min(1),
  max_tokens: z.number().optional().default(50),
});

export const POST = router
  .route({ path: 'v1/chat/completions' })
  .paid('0.05')
  .body(ChatCompletionSchema)
  .description('Send a message to Cathedral ARKHE OpenAI Gateway.')
  .handler(async ({ body, request }) => {
    const backendUrl = 'http://127.0.0.1:8080/v1/chat/completions';
    try {
      const response = await fetch(backendUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        throw Object.assign(new Error('Backend error'), { status: response.status });
      }

      const data = await response.json();
      return data;
    } catch (error: any) {
      if (error.status) throw error;
      throw Object.assign(new Error('Failed to reach backend'), { status: 502 });
    }
  });
