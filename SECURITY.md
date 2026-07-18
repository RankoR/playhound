# Security policy

## Supported versions

Security fixes are provided for the latest released version.

## Reporting a vulnerability

Please use GitHub's private vulnerability-reporting feature for the
`RankoR/playhound` repository. Do not open a public issue for an unpatched
vulnerability. Include affected versions, impact, reproduction steps, and any
suggested mitigation. You should receive an acknowledgement within seven days.

PlayHound scrapes an unsupported web interface. A parsing failure caused only by
an upstream layout change is normally a compatibility bug, not a vulnerability.
Credential disclosure, proxy bypass, request smuggling, unsafe URL handling, and
denial-of-service through unbounded input are security concerns.

