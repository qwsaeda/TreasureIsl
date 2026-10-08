/* ea:   0x0003DFF0 */
/* name: sub_3DFF0 */
/* size: 96 */
int __fastcall sub_3DFF0(int *a1, _DWORD *a2)
{
  char *v4; // r1
  int result; // r0
  char v6; // [sp+8h] [bp-Ch] BYREF

  v4 = (char *)a1[1];
  if ( v4 == (char *)a1[2] )
    return sub_38118(a1, v4, a2, (int)&v6, 1, 1);
  if ( v4 != nullptr )
    sub_132FC(v4, a2);
  result = a1[1];
  a1[1] = result + 12;
  return result;
}

