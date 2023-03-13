import "./style.css";
import * as z from "zod";
// import { Watchout } from "./api";

// const btn = document.getElementById("btn");
// const resField = document.getElementById("res");
// const streamField = document.getElementById("stream-value");
// const streamSwitch = document.getElementById("stream-switch");

// let unsubscribe: (() => void) | undefined = undefined;

// btn &&
// btn.addEventListener("click", async () => {
// // let pix: Pixera.MyEntity2 = { value: 123 };
// let res = await Watchout.hello("abc", 123);
// if (resField) {
// resField.innerHTML = res.toString();
// }
// });

// streamSwitch &&
// streamSwitch.addEventListener("change", (evt) => {
// const target: (EventTarget & { checked: boolean }) | null =
// evt.target as any;
// const isChecked = target && target.checked;

// if (isChecked) {
// let stream = Watchout.hello_stream(10);
// unsubscribe = stream.subscribe((val) => {
// if (streamField) {
// streamField.innerHTML = val.toString();
// }
// });
// } else {
// unsubscribe && unsubscribe();
// }
// });

export namespace A {
  export const a = z.number();
  export const b = z.object({
    b: z.lazy(() => B.b),
  });

  export const c = z.object({
    c: z.lazy(() => A.b),
  });
}

export namespace B {
  export const b = z.lazy(() => A.a);
}

A.b.parse({ b: { b: { b: { b: 123 } } } });
