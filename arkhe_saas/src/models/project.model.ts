// src/models/project.model.ts
import {Entity, model, property, belongsTo} from '@loopback/repository';
import {User} from './user.model';

@model({settings: {postgresql: {table: 'projects'}}})
export class Project extends Entity {
  @property({id: true, generated: true})
  id: string;

  @property({type: 'string', required: true})
  name: string;

  @belongsTo(() => User)
  createdBy: string;

  constructor(data?: Partial<Project>) {
    super(data);
  }
}
