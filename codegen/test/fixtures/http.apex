namespace "http-request"

type Inputs {
  request: HttpRequest
}

type Outputs {
  response: HttpResponse
}

type HttpResponse {
  body:string
  headers: {string: string}
}

type HttpRequest {
  url: string
  method: string
  link: link @capability("http")
}