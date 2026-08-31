// src/models/user.model.ts
import {Entity, model, property} from '@loopback/repository';

@model({settings: {postgresql: {table: 'users'}}})
export class User extends Entity {
  @property({id: true, generated: true})
  id: string;

  @property({type: 'string', required: true})
  email: string;

  @property({type: 'string', required: true})
  name: string;

  constructor(data?: Partial<User>) {
    super(data);
  }
}
