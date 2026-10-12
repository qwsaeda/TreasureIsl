/* ea:   0x00060118 */
/* name: sub_60118 */
/* size: 52 */
int __fastcall sub_60118(int a1, int a2, int a3)
{
  int v4; // r3
  char v6; // [sp+0h] [bp-10h] BYREF

  v4 = sub_60C54(a1, &v6, a3);
  *(_DWORD *)a2 = *(_DWORD *)v4;
  *(_BYTE *)(a2 + 4) = *(_BYTE *)(v4 + 4);
  return a2;
}

