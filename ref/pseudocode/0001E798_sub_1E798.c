/* ea:   0x0001E798 */
/* name: sub_1E798 */
/* size: 88 */
int __fastcall sub_1E798(_DWORD *a1)
{
  int result; // r0

  *a1 = off_6D1E4;
  sub_63C88(a1);
  a1[5] = off_6D220;
  sub_620B0(a1 + 5);
  a1[5] = &off_6D248;
  result = sub_1E81C(a1 + 2);
  *a1 = &off_6D270;
  return result;
}

