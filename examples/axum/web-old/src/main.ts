import "./style.css";
import { Rs, Pixera, Watchout } from "./api";

// const btn = document.getElementById("btn");
// const resField = document.getElementById("res");
// const streamField = document.getElementById("stream-value");
// const streamSwitch = document.getElementById("stream-switch");

// let unsubscribe: (() => void) | undefined = undefined;

class Client implements Rs.Client {
  addr: string;

  constructor({ addr }: { addr: string }) {
    this.addr = addr;
  }

  request(ns: string, name: string, args: IArguments) {
    console.log({ addr: this.addr, ns, name, args });
  }

  subscribe(ns: string, name: string, args: IArguments): Rs.Store<undefined> {
    console.log({ addr: this.addr, ns, name, args });
    return {
      subscribe() {
        return () => {};
      },
      close() {},
    };
  }

  init<T extends { [k: string]: any }>(ns: T): Mapped<Filtered<T>> {
    const out: Partial<Mapped<Filtered<typeof Pixera>>> = {};

    Object.entries(ns).forEach(([k, _]) => {
      let item = ns[k as keyof T];
      if ((item as any) instanceof Function) {
        let f = item as Function;
        out[k as keyof Mapped<Filtered<T>>] = (...args: unknown[]) =>
          f(...args);
      }
    });

    return out as Mapped<Filtered<T>>;
  }
}

type AnyMethod<T> = T extends (...args: any[]) => any ? T : never;

type Tail<T extends any[]> = T extends [infer _, ...infer R] ? R : never;

type Mapped<T> = {
  -readonly [K in keyof T]: T[K] extends (...args: any[]) => any
    ? MappedMethod<T[K]>
    : undefined;
};

type Filtered<Source> = Pick<
  Source,
  {
    [K in keyof Source]: Source[K] extends AnyMethod<Source[K]> ? K : never;
  }[keyof Source]
>;

type MappedMethod<T> = T extends (...args: any[]) => infer K
  ? (...args: Tail<Parameters<T>>) => K
  : never;

let client = new Client({ addr: "ws://localhost:8000/sdk" });
let watchout = client.init(Watchout);

let res = watchout.hello("ab", 2);

// btn &&
// btn.addEventListener("click", async () => {
// // let pix: Pixera.MyEntity2 = { value: 123 };
// let res = await watchout.hello(client, "abc", 123);
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
// let stream = watchout.hello_stream(client, 10);
// unsubscribe = stream.subscribe((val) => {
// if (streamField) {
// streamField.innerHTML = val.toString();
// }
// });
// } else {
// unsubscribe && unsubscribe();
// }
// });
