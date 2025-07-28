import { dedupe } from "oveo";

dedupe({ a: x`ab${c}d` });
dedupe({ a: x`ab${c}d` });
dedupe({ a: x`ad${c}b` });
