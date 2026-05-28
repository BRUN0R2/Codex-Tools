export type CommandFailure = Readonly<{
  code: string;
  message: string;
}>;

export type CommandResult<Value> =
  | Readonly<{
      ok: true;
      value: Value;
    }>
  | Readonly<{
      ok: false;
      error: CommandFailure;
    }>;
