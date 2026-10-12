/* ea:   0x0004D380 */
/* name: sub_4D380 */
/* size: 324 */
int __fastcall sub_4D380(int a1, int a2)
{
  int v4; // r0
  int v5; // r4
  unsigned int v7; // r0
  _DWORD *v8; // r1
  unsigned int v9; // r0

  if ( a2 != 0 && *(unsigned __int8 *)(a2 + 50) == *(_DWORD *)(a1 + 8) )
  {
    v4 = *(unsigned __int8 *)(a2 + 49);
    v5 = 0;
    if ( v4 == 1 )
    {
      if ( (unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 2) >= 0x7D0 )
      {
        v8 = (_DWORD *)(a1 + 144);
        v9 = *(_DWORD *)(a1 + 144);
        if ( v9 <= 0x7D0 )
          goto LABEL_10;
        *v8 = v9 - 2000;
        sub_17BC8(a2, (_DWORD *)(*(_DWORD *)(a1 + 24) + 32));
        return 1;
      }
    }
    else
    {
      if ( v4 != 2 )
      {
        if ( v4 == 3 && (unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 2) >= 0x2710 )
        {
          sub_16570((unsigned int *)(a1 + 144), 0x2710u);
          sub_17BC8(a2, (_DWORD *)(*(_DWORD *)(a1 + 24) + 32));
          return 1;
        }
        return v5;
      }
      if ( (unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 2) >= 0xFA0 )
      {
        v8 = (_DWORD *)(a1 + 144);
        v7 = *(_DWORD *)(a1 + 144);
        if ( v7 > 0xFA0 )
        {
          *v8 = v7 - 4000;
          sub_17BC8(a2, (_DWORD *)(*(_DWORD *)(a1 + 24) + 32));
          return 1;
        }
LABEL_10:
        *v8 = 0;
        sub_17BC8(a2, (_DWORD *)(*(_DWORD *)(a1 + 24) + 32));
        return 1;
      }
    }
    return v5;
  }
  return 0;
}

