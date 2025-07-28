import { dedupe } from "oveo";

dedupe({ a: 1n });
dedupe({ a: 1n });
dedupe({ a: 2n });
