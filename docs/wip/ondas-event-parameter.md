# Zero-width VCD parameter investigation

Ondas issue [18](https://github.com/kleverhq/ondas/issues/18) is withdrawn as a migration blocker.

The reproduction demonstrates a difference from Wellen, not a demonstrated producer compatibility requirement. Wellen interprets zero-width bit vectors as events and zero-width parameters as event parameters. The only identified Wavepeek input exercising this behavior is a synthetic unit-test fixture; no real producer or dump has been identified.

IEEE 1364-2001 §18.2.3.8 (printed page 335) lists `parameter` and `event` separately in the VCD declaration syntax. It does not specify that a zero-width parameter denotes an event. The earlier description as a producer/reader compatibility encoding was unsupported by the evidence.

Following maintainer clarification, the synthetic fixture now declares `$var event 1 % second_event $end`. Its checks still require event type resolution and the occurrence at tick 5, alongside the existing real, string, ordinary-event, and non-event checks. Ondas already supports this explicit event declaration.

The original comparison logs remain under `tmp/ondas-migration/` as evidence of reader behavior only. They are not grounds for requiring an Ondas extension. If a real producer example is found later, compatibility support can be assessed separately and documented narrowly.
