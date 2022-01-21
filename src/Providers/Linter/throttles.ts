// A class to define functions that can detect and deal with timeout, if spcomp crashes.
export class TimeoutFunction {
  private timeout: NodeJS.Timer;

  constructor() {
    this.timeout = null;
  }

  public start(callback: (...args: any[]) => void, delay: number) {
    this.timeout = setTimeout(callback, delay);
  }

  public cancel() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = undefined;
    }
  }
}

export const throttles: { [key: string]: TimeoutFunction } = {};
