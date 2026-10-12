/* ea:   0x00044AFC */
/* name: sub_44AFC */
/* size: 108 */
int __fastcall sub_44AFC(int a1)
{
  int v2; // r0

  *(_WORD *)(a1 + 20) = -1;
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)(a1 + 8) = 0;
  *(_DWORD *)(a1 + 12) = 0;
  *(_DWORD *)(a1 + 16) = 0;
  *(_WORD *)(a1 + 22) = 0;
  *(_WORD *)(a1 + 24) = 0;
  *(_WORD *)(a1 + 26) = 0;
  *(_DWORD *)(a1 + 28) = 0;
  v2 = a1 + 28;
  *(_DWORD *)(v2 + 4) = 0;
  *(_DWORD *)(v2 + 8) = 0;
  *(_BYTE *)(a1 + 40) = 0;
  *(_BYTE *)(a1 + 41) = 0;
  *(_DWORD *)(a1 + 44) = 0;
  *(_DWORD *)a1 = off_6FD58;
  *(_BYTE *)(a1 + 48) = 0;
  *(_BYTE *)(a1 + 49) = 0;
  *(_BYTE *)(a1 + 50) = 0;
  return a1;
}

