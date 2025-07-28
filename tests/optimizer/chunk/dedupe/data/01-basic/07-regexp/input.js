import { dedupe } from "oveo";

dedupe({ a: /a/g });
dedupe({ a: /a/g });
dedupe({ a: /a/gi });
dedupe({ a: /b/g });
