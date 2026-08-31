// src/controllers/handover.controller.ts
import {inject} from '@loopback/core';
import {
  post,
  get,
  param,
  requestBody,
  response,
  RestBindings,
  Response,
} from '@loopback/rest';
import {repository} from '@loopback/repository';
import {HandoverRepository} from '../repositories/handover.repository';
import {Handover} from '../models';
import {SqsService} from '../services/sqs.service';
import {WebhookService} from '../services/webhook.service';
import {authenticate} from '@loopback/authentication';
import {RedisService} from '../services/redis.service';

@authenticate('jwt') // OAuth 2.0 protege todos os endpoints
export class HandoverController {
  constructor(
    @repository(HandoverRepository)
    public handoverRepository: HandoverRepository,
    @inject('services.SqsService')
    public sqsService: SqsService,
    @inject('services.WebhookService')
    public webhookService: WebhookService,
    @inject('services.RedisService')
    public redisService: RedisService,
  ) {}

  @post('/handovers')
  @response(201, {description: 'Novo handover registrado'})
  async createHandover(
    @requestBody() handover: Omit<Handover, 'id'>,
    @inject(RestBindings.Http.RESPONSE) response: Response,
  ): Promise<Handover> {
    // 1. Salva no PostgreSQL (WormGraph)
    const saved = await this.handoverRepository.create(handover);

    // 2. Publica no SQS para processamento assíncrono (JAX/LLM)
    await this.sqsService.sendMessage({
      type: 'HANDOVER_CREATED',
      handoverId: saved.id,
      projectId: saved.projectId,
      coherence: saved.coherence,
    });

    // 3. Dispara webhooks para integrações externas (Altium, ERP)
    await this.webhookService.trigger('handover.created', {
      id: saved.id,
      coherence: saved.coherence,
      projectId: saved.projectId,
    });

    // 4. Atualiza cache Redis (para monitoramento em tempo real)
    // Note: redisService.set expects `any` and stringifies internally, we pass the object directly.
    await this.redisService.set(`handover:${saved.id}`, saved, 3600);

    return saved;
  }

  @get('/handovers/project/{projectId}')
  @response(200, {description: 'Lista de handovers de um projeto'})
  async findByProject(
    @param.path.string('projectId') projectId: string,
    @param.query.number('limit') limit: number = 100,
  ): Promise<Handover[]> {
    return this.handoverRepository.find({
      where: {projectId},
      order: ['timestamp DESC'],
      limit,
    });
  }

  @get('/handovers/coherence-range')
  @response(200, {description: 'Handovers com coerência acima do limiar'})
  async findByCoherence(@param.query.number('min') min: number = 0.94): Promise<Handover[]> {
    return this.handoverRepository.find({
      where: {coherence: {gte: min}},
      order: ['timestamp DESC'],
    });
  }
}
