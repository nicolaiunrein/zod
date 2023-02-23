import "./style.css";
import { Watchout } from "./api";

const btn = document.getElementById("btn");
const resField = document.getElementById("res");
const streamField = document.getElementById("stream-value");
const streamSwitch = document.getElementById("stream-switch");

let unsubscribe: (() => void) | undefined = undefined;

btn &&
  btn.addEventListener("click", async () => {
    let res = await Watchout.nested({ value: { value: 123 } });
    if (resField) {
      resField.innerHTML = res.toString();
    }
  });

streamSwitch &&
  streamSwitch.addEventListener("change", (evt) => {
    const target: (EventTarget & { checked: boolean }) | null =
      evt.target as any;
    const isChecked = target && target.checked;

    if (isChecked) {
      let stream = Watchout.hello_stream(10);
      unsubscribe = stream.subscribe((val) => {
        if (streamField) {
          streamField.innerHTML = val.toString();
        }
      });
    } else {
      unsubscribe && unsubscribe();
    }
  });

const x: Watchout.MyEntity = { value: 123 };

// console.log(C.c);
