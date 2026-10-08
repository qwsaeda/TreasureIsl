/* ea:   0x0001CBF8 */
/* name: sub_1CBF8 */
/* size: 176 */
int __fastcall sub_1CBF8(int a1, int a2, int a3, int a4)
{
  int v8; // t0

  sub_4EC88();
  *(_DWORD *)(a1 + 48) = a2;
  *(_DWORD *)(a1 + 52) = a4;
  *(_DWORD *)(a1 + 56) = 56 - *(_DWORD *)(a4 + 12);
  *(_DWORD *)(a1 + 60) = 94 - *(_DWORD *)(a4 + 16);
  *(_DWORD *)(a1 + 64) = (int)*(unsigned __int16 *)(a4 + 20) % *(_DWORD *)(a2 + 692);
  v8 = *(unsigned __int16 *)(a4 + 20) / *(int *)(a2 + 692);
  *(_DWORD *)(a1 + 72) = a3;
  *(_DWORD *)(a1 + 68) = v8;
  *(_BYTE *)(a1 + 84) = 0;
  *(_BYTE *)(a1 + 85) = 0;
  *(_DWORD *)(a1 + 88) = 0;
  *(_DWORD *)(a1 + 92) = 0;
  *(_DWORD *)a1 = off_6D1C0;
  return a1;
}

