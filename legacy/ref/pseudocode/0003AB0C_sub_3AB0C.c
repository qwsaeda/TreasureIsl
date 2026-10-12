/* ea:   0x0003AB0C */
/* name: sub_3AB0C */
/* size: 56 */
int __fastcall sub_3AB0C(int a1, _WORD *a2, int a3)
{
  wchar_t v6[22]; // [sp+0h] [bp-2Ch] BYREF

  itow(a3, v6, 10);
  return sub_3AA9C(a1, a2, (int)v6);
}

