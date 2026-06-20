import { createRouterFromEnv } from '@agentcash/router';

export const router = createRouterFromEnv({
  title: 'Cathedral ARKHE API',
  description: 'Cathedral ARKHE provides paid routes for interacting with its models.',
  guidance: 'Cathedral ARKHE provides APIs for chat completions. Endpoint requires payment.',
  contact: { name: 'Cathedral ARKHE', url: 'https://cathedral-arkhe.io' },
  strictRoutes: false,
});
