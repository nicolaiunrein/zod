import * as z from "zod";
export const MyNullSchema = z.object({}).brand<"valid">();
type MyNull = z.infer<typeof MyNullSchema>;

export let my_null: MyNull = MyNullSchema.parse(null);

function test(_value: MyNull) {}
test(my_null);
test({});
