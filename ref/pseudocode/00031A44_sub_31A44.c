/* ea:   0x00031A44 */
/* name: sub_31A44 */
/* size: 184 */
int __fastcall sub_31A44(int a1)
{
  _DWORD *v2; // r4
  char v4; // [sp+0h] [bp-18h] BYREF
  _BYTE v5[3]; // [sp+1h] [bp-17h] BYREF
  unsigned __int8 v6; // [sp+4h] [bp-14h] BYREF

  v2 = (_DWORD *)(a1 + 32);
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)(a1 + 8) = 0;
  *(_DWORD *)(a1 + 28) = 0;
  sub_31AFC(a1 + 32, v5, &v4);
  v6 = 0;
  *(_DWORD *)a1 = off_6DAF8;
  *(_BYTE *)(a1 + 17) = 4;
  *(_BYTE *)(a1 + 18) = 4;
  *(_BYTE *)(a1 + 20) = 4;
  *(_BYTE *)(a1 + 22) = 4;
  *(_DWORD *)(a1 + 44) = 100;
  *(_BYTE *)(a1 + 16) = 6;
  *(_BYTE *)(a1 + 19) = 6;
  *(_BYTE *)(a1 + 21) = 6;
  *(_BYTE *)(a1 + 23) = 6;
  *(_BYTE *)(a1 + 25) = 5;
  *(_BYTE *)(a1 + 24) = 5;
  *(_BYTE *)(a1 + 27) = 5;
  *(_BYTE *)(a1 + 26) = 5;
  do
  {
    *(_BYTE *)sub_318D0(v2, &v6) = 0;
    ++v6;
  }
  while ( v6 < 0xAu );
  return a1;
}

