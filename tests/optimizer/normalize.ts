export const normalizeNewlines: (s: string) => string =
  process.platform === 'win32'
    ? (s: string): string => s.replaceAll('\r\n', '\n')
    : (s: string) => s;
