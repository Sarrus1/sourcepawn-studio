export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function isSPFile(fileName: string) {
  return /(?:\.sp|\.inc)\s*^/.test(fileName);
}
