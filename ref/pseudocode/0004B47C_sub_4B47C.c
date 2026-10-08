/* ea:   0x0004B47C */
/* name: sub_4B47C */
/* size: 160 */
int __fastcall sub_4B47C(int a1, int a2)
{
  int result; // r0
  unsigned __int16 v5; // r1
  int v6; // r2

  result = *(_DWORD *)(a1 + 20);
  if ( result != 0 )
  {
    *(_DWORD *)a2 = 1;
    *(_WORD *)(a2 + 16) = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    *(_WORD *)(a2 + 18) = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    *(_DWORD *)(a2 + 12) = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
    v5 = *(_WORD *)(*(_DWORD *)(a1 + 20) + 26);
    *(_DWORD *)(a2 + 4) = 1;
    *(_DWORD *)(a2 + 28) = v5;
    v6 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 84);
    *(_DWORD *)(a2 + 20) = 0;
    *(_DWORD *)(a2 + 8) = v6;
    *(_WORD *)(a2 + 24) = 0;
    *(_WORD *)(a2 + 26) = 0;
    return 0;
  }
  return result;
}

