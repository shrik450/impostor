#! /usr/bin/env impostor

# This is a tour of Impostor in code.
#
# Impostor is a mock server that uses a simple, text based format to define
# mocks.
#
# As you can see, lines starting with a hash are comments.

# Let's define our first mock:

GET /
# This is the request line. It must have the method and path. The path shouldn't
# have any query string.
HTTP 200
`hello, world!`
# This is the response. It must have HTTP followed by the status code. You can
# optionally add a body. In this case, the body is a single line string: hello,
# world!. The backticks are used to delimit the string, and won't be present in
# the response.

# You define multiple routes in the same file. Let's define another one:

POST /
HTTP 201
# Let's add a header to the response:
Content-Type: application/json
# For JSON bodies, you don't need to quote the response in backticks.
{"id": 1}

# And some more:

PUT /
HTTP 200
`updated`

DELETE /
HTTP 204

# As you can see, the body is optional.

# If you want to match requests based on parameters in the request, you can use
# Asserts. Asserts are defined in the request section:

GET /users
[Asserts]
queryparam "name" == "John"

HTTP 200
{"name": "John"}

# This will match GET requests to /users?name=John. If the request doesn't have
# the name parameter, or if it has a different value, the mock won't match, and
# Impostor will return a 404.

# Apart from query params, you can match on:
#
# - Headers with the `header` assert.
# - Cookies with the `cookie` assert.
# - The full path of the request with the `path` assert.
#
# You can also implicitly define a header assert:

GET /orders
Accept: application/json
[Asserts]
queryparam "id" == "123"

HTTP 200
Content-Type: application/json
{"id": 123}

# This will match GET requests to /users?name=John with the Accept header set to
# application/json.

# You can also define multiple mocks to the same route with different asserts,
# such as:

GET /orders
Accept: text/html
[Asserts]
queryparam "id" == "123"

HTTP 200
Content-Type: text/html
`<html><body>Order 123</body></html>`

# This will match GET requests to /users?name=John with the Accept header set to
# text/html.

# If there are multiple routes that match a request, Impostor will return the
# first one defined in the file. So, if you want to have a fallback route, you
# should define it last. Here is a somewhat contrived example:

GET /offers
[Asserts]
queryparam "id" == "1"

HTTP 200
Content-Type: application/json
{"id": 1, "name": "Offer 1"}

GET /offers
[Asserts]
queryparam "id" == "2"

HTTP 200
Content-Type: application/json
{"id": 2, "name": "Offer 2"}

GET /offers
[Asserts]
queryparam "id" exists

HTTP 404
Content-Type: application/json
{"error": "Offer not found"}

GET /offers
HTTP 400

Content-Type: application/json
{"error": "Offer ID must be present"}

# This does what you expect it to: it returns the offer with the given ID, or a
# 404 if it doesn't exist, or a 400 if the ID is missing.
