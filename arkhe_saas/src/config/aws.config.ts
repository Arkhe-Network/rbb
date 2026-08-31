// src/config/aws.config.ts
export const config = {
  aws: {
    region: process.env.AWS_REGION || 'us-east-1',
    credentials: {
      accessKeyId: process.env.AWS_ACCESS_KEY_ID || 'mock',
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY || 'mock',
    },
    sqs: {
      handoverQueueUrl: process.env.SQS_QUEUE_URL || 'http://localhost:4566/000000000000/handover-queue',
    }
  }
};
