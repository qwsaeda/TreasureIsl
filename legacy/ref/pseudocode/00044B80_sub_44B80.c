/* ea:   0x00044B80 */
/* name: sub_44B80 */
/* size: 232 */
int __fastcall sub_44B80(int a1, int a2, int a3, int a4)
{
  unsigned __int16 v6; // r0
  int v7; // r0
  int v8; // r1
  int v10; // [sp+0h] [bp-1Ch] BYREF

  *(_DWORD *)(a1 + 4) = a4;
  *(_WORD *)(a1 + 20) = *(_WORD *)(a2 + 16);
  v6 = *(_WORD *)(a1 + 20);
  *(_DWORD *)(a1 + 44) = (unsigned int)(*(unsigned __int8 *)(a2 + 15) << 26) >> 31;
  v8 = v6 % a3;
  v7 = v6 / a3;
  *(_DWORD *)(a1 + 12) = ((v7 + v8) << 6) / 2;
  *(_DWORD *)(a1 + 16) = (32 * (v7 - v8 + a3) - 32) / 2;
  *(_WORD *)(a1 + 26) = *(_WORD *)(a2 + 22);
  LOWORD(v8) = *(_WORD *)(a2 + 18);
  *(_BYTE *)(a1 + 40) = 1;
  *(_BYTE *)(a1 + 41) = 0;
  *(_BYTE *)(a1 + 48) = v8 & 0xF;
  sub_4518C(a1 + 28, 5);
  v10 = 0;
  sub_450F8(a1 + 28, 5, &v10);
  return 1;
}

