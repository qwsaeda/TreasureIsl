/* ea:   0x000605D4 */
/* name: sub_605D4 */
/* size: 64 */
void **__fastcall sub_605D4(void **a1, int a2)
{
  int v3; // r0
  wchar_t v5[68]; // [sp+0h] [bp-88h] BYREF

  itow(a2, v5, 10);
  v3 = sub_12F04(v5);
  return sub_1318C(a1, (char *)v5, (int)&v5[v3]);
}

