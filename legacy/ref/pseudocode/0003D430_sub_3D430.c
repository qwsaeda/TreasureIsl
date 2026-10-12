/* ea:   0x0003D430 */
/* name: sub_3D430 */
/* size: 108 */
int __fastcall sub_3D430(int a1)
{
  *(_DWORD *)a1 = -1;
  *(_BYTE *)(a1 + 32) = 1;
  *(_BYTE *)(a1 + 33) = 1;
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)(a1 + 8) = 0;
  *(_BYTE *)(a1 + 12) = 0;
  *(_DWORD *)(a1 + 16) = 0;
  *(_DWORD *)(a1 + 20) = 0;
  *(_DWORD *)(a1 + 24) = 0;
  *(_DWORD *)(a1 + 28) = 0;
  sub_1333C((_DWORD *)(a1 + 36));
  *(_DWORD *)(a1 + 48) = 0;
  *(_DWORD *)(a1 + 52) = 0;
  *(_DWORD *)(a1 + 56) = 0;
  *(_DWORD *)(a1 + 60) = 0;
  off_811A0 = (_UNKNOWN *)a1;
  return a1;
}

