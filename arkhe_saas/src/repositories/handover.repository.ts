// src/repositories/handover.repository.ts
import {inject} from '@loopback/core';
import {DefaultCrudRepository} from '@loopback/repository';
import {Handover} from '../models';

export class HandoverRepository extends DefaultCrudRepository<
  Handover,
  typeof Handover.prototype.id
> {
  constructor(
    @inject('datasources.postgresql') dataSource: any,
  ) {
    super(Handover, dataSource);
  }
}
