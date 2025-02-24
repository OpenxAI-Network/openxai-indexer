import storage from "node-persist";

import { replacer, reviver } from "./json.js";

export class PersistentJson<T> {
  private storageKey: string;
  private value?: T;
  private defaultValue: T;

  constructor(storageKey: string, defaultValue: T) {
    this.storageKey = storageKey;
    this.value = undefined;
    this.defaultValue = defaultValue;
  }

  public async get(): Promise<T> {
    if (this.value === undefined) {
      const storedValue = await storage.getItem(this.storageKey);
      if (this.value === undefined) {
        // Could have been loaded by someone else in the meanwhile
        // Default value is return directly, meaning that is might be altered (currently not an issue)
        this.value = storedValue
          ? (JSON.parse(storedValue, reviver) as T)
          : this.defaultValue;
      }
    }

    return this.value;
  }

  public async update(update: (currentValue: T) => void): Promise<void> {
    const value = await this.get();
    // Edit the same object to prevent concurrency issues
    update(value);
    await storage.setItem(this.storageKey, JSON.stringify(value, replacer));
  }
}
