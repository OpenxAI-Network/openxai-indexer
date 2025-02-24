export interface Filter {
  min?: number | bigint;
  max?: number | bigint;
  equal?: string | number | bigint;
  includes?: string | any;
  some?: Filter;
  oneOf?: Filter[];
  not?: Filter;
  objectFilter?: ObjectFilter;

  convertValueToLowercase?: boolean;
}

export interface ObjectFilter {
  [property: string]: Filter;
}

function typeCheck(type: string, filterType: string) {
  if (filterType !== type) {
    throw new Error(`Type mismatch (filter: ${filterType}, value: ${type})`);
  }
}

export function passesFilter(value: any, filter: Filter): boolean {
  if (value === undefined) {
    return false;
  }
  if (filter.convertValueToLowercase) {
    if (typeof value === "string") {
      value = value.toLowerCase();
    } else {
      throw new Error(`ConvertValueToLowercase filter on non string value ${value}`);
    }
  }

  if (filter.min !== undefined) {
    typeCheck(typeof value, typeof filter.min);
    if (value < filter.min) {
      return false;
    }
  }
  if (filter.max !== undefined) {
    typeCheck(typeof value, typeof filter.max);
    if (value > filter.max) {
      return false;
    }
  }
  if (filter.equal !== undefined) {
    typeCheck(typeof value, typeof filter.equal);
    if (value !== filter.equal) {
      return false;
    }
  }
  if (filter.includes !== undefined) {
    if (!value.includes(filter.includes)) {
      return false;
    }
  }
  if (filter.some !== undefined) {
    const someFilter = filter.some; // To help typescript inference that this is not undefined
    if (!value.some((v: any) => passesFilter(v, someFilter))) {
      return false;
    }
  }
  if (filter.oneOf !== undefined) {
    if (!filter.oneOf.some((f) => passesFilter(value, f))) {
      return false;
    }
  }
  if (filter.not !== undefined) {
    if (passesFilter(value, filter.not)) {
      return false;
    }
  }
  if (filter.objectFilter !== undefined) {
    return passesObjectFilter(value, filter.objectFilter);
  }

  return true;
}

export function passesObjectFilter(object: any, objectFilter: ObjectFilter): boolean {
  const properties = Object.keys(objectFilter);
  for (let i = 0; i < properties.length; i++) {
    const property = properties[i];
    if (!passesFilter(object[property], objectFilter[property])) {
      return false;
    }
  }

  return true;
}
