# Test scripts

Development helpers. Not part of the application.

## seed-test-contacts.sql

Inserts 20 fake contacts with phone numbers `+15551000001` through
`+15551000020`. They are tagged `test` and can be removed at any time:

    docker exec -i asterisk-pg psql -U callcenter -d callcenter \
      < /home/thomas/Coding/Rust/call-me/scripts/seed-test-contacts.sql

To remove them later:

    docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
      "DELETE FROM contacts WHERE 'test' = ANY(tags);"

## simulate-call.sh

Originates a fake inbound call to a queue. Uses AMI to create a call
with a chosen caller ID and routes it through the dialplan.

    ./simulate-call.sh                    # random number, queue 2000
    ./simulate-call.sh +15551000007       # specific number
    ./simulate-call.sh +15551000007 2001  # specific queue

The recipient hears a short "hello world" playback. Answer it, then
hang up. The call is real: it appears in `cdr`, `cel`, the agent's call
history, and (if MixMonitor is running) it is recorded.

Requires `nc` (netcat).
