/* ea:   0x0002F9D8 */
/* name: sub_2F9D8 */
/* size: 52 */
int __fastcall sub_2F9D8(int a1, int a2)
{
  int v3; // r3
  char v5; // [sp+0h] [bp-10h] BYREF

  v3 = sub_37468(a1, &v5);
  *(_DWORD *)a2 = *(_DWORD *)v3;
  *(_BYTE *)(a2 + 4) = *(_BYTE *)(v3 + 4);
  return a2;
}

