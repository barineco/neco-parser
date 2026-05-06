lex "encoding.base64.encode" {
    procedure {
        step "encoding.base64.decode"
    }
}

cratis "encoding" version=1 {
    provides { item "encoding.base64.encode" }
    requires { item "bytes" }
}

morph "crypto.jwt.issue" {
    requires output="claims"
    produces output="token"
}

morph.derives "str.contains" via="compose" {
    sources { item "str.find" }
    steps { step "str.find" }
    returns "bool"
}

func.family "Numeric" {
    members { item "i32.add" }
    signature "closed"
}

law "arith_comm" {
    kind "commutative"
    over "i32.add"
}

inverse "mute_actor" {
    action "actor.mute"
    inverse "actor.unmute"
    kind "reversible"
}

dual "follow_dual" {
    record "graph.follow"
    forward "graph.follow"
    reverse "graph.unfollow"
}

handler "ink.illo.dm.send" {
    chain {
        step "encoding.base64.encode" input="raw" output="encoded"
        .step "crypto.jwt.issue" input="encoded" output="token"
    }
}

chain "standalone.pipeline" {
    step "str.find"
}

import "neco-vault" {
    procedure "vault.open" {
        in { item "path" }
        out { item "handle" }
    }
}

lexicon "app.bsky.actor.getProfile" version=1 xrpc="query" {
    output "app.bsky.actor.defs#profileViewDetailed"
}

face "client" {
    emit "client.event"
    axiom "encoding.base64.encode"
}
