namespace "key-set"

type Inputs {
  "The key to set."
  key: string,

  "The value to set."
  value: string,

  "The expiration in seconds for the key."
  expires: u32,
}

type Outputs {
  "The result of the transaction."
  result: bool
}
