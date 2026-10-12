/* ea:   0x00055A10 */
/* name: sub_55A10 */
/* size: 388 */
int __fastcall sub_55A10(int a1)
{
  int v2; // r0
  int v3; // r7
  int result; // r0
  unsigned int v5; // r3
  bool v6; // zf
  int v7; // r1
  _DWORD *v8; // r4
  unsigned __int8 v9; // r5
  int v10; // r0
  int v11; // [sp+0h] [bp-1Ch] BYREF
  int v12; // [sp+4h] [bp-18h] BYREF

  v2 = *(_DWORD *)(a1 + 208);
  v3 = 0;
  if ( v2 == 1 )
  {
    if ( *(_DWORD *)(a1 + 12) != 0 )
    {
      result = *(_DWORD *)(a1 + 24);
      if ( *(_WORD *)(a1 + 212) == 1 )
      {
        v7 = *(_DWORD *)(result + 300);
        LOBYTE(result) = 1;
        if ( *(_DWORD *)(a1 + 12) != v7 )
          return 0;
        return (unsigned __int8)result;
      }
      v8 = *(_DWORD **)(result + 496);
      v11 = *(unsigned __int16 *)(a1 + 212);
      v9 = 1;
      if ( sub_1EDAC(result + 496, &v11) == v8 )
        return 0;
      v10 = *(_DWORD *)(a1 + 24);
      v12 = *(unsigned __int16 *)(a1 + 212);
      if ( *(_DWORD *)(a1 + 12) != *(_DWORD *)(*(_DWORD *)sub_1DD10((_DWORD *)(v10 + 496), &v12) + 12) )
        return 0;
      return v9;
    }
  }
  else
  {
    if ( v2 != 2 )
    {
      if ( v2 == 3 )
      {
        result = *(_DWORD *)(a1 + 12);
        if ( result != 0 )
        {
          if ( (*(_BYTE *)(result + 22) & 0x3F) != 5
            || (v5 = (*(_WORD *)(result + 24) & 0xC0u) >> 6, LOBYTE(result) = 1, v5 != *(unsigned __int16 *)(a1 + 212)) )
          {
            LOBYTE(result) = 0;
          }
          return (unsigned __int8)result;
        }
      }
      return v3;
    }
    result = *(_DWORD *)(a1 + 20);
    if ( result != 0 )
    {
      v6 = *(unsigned __int16 *)(a1 + 212) == *(unsigned __int16 *)(result + 20);
      LOBYTE(result) = 1;
      if ( !v6 )
        return 0;
      return (unsigned __int8)result;
    }
  }
  return v3;
}

