/* ea:   0x000183FC */
/* name: sub_183FC */
/* size: 332 */
_BYTE *__fastcall sub_183FC(_BYTE *a1, _DWORD *a2)
{
  _BYTE *v2; // r4
  int v4; // r2
  int v5; // r0
  _BYTE *v6; // r1
  _BYTE *v7; // r1
  _BYTE *result; // r0

  v2 = a1;
  *a1 = 0;
  while ( v2 != (_BYTE *)*a2 )
  {
    v4 = *((_DWORD *)v2 + 1);
    if ( *(_BYTE *)v4 != 0 )
      break;
    v5 = *(_DWORD *)(v4 + 4);
    v6 = *(_BYTE **)(v5 + 8);
    if ( (_BYTE *)v4 == v6 )
    {
      v7 = *(_BYTE **)(v5 + 12);
      if ( v7 == nullptr || *v7 != 0 )
      {
        if ( v2 == *(_BYTE **)(v4 + 12) )
        {
          v2 = *((_BYTE **)v2 + 1);
          sub_179A0(v4, a2);
        }
        **((_BYTE **)v2 + 1) = 1;
        **(_BYTE **)(*((_DWORD *)v2 + 1) + 4) = 0;
        sub_17A18(*(_DWORD *)(*((_DWORD *)v2 + 1) + 4), a2);
      }
      else
      {
        *(_BYTE *)v4 = 1;
        *v7 = 1;
        **(_BYTE **)(*((_DWORD *)v2 + 1) + 4) = 0;
        v2 = *(_BYTE **)(*((_DWORD *)v2 + 1) + 4);
      }
    }
    else if ( v6 == nullptr || *v6 != 0 )
    {
      if ( v2 == *(_BYTE **)(v4 + 8) )
      {
        v2 = *((_BYTE **)v2 + 1);
        sub_17A18(v4, a2);
      }
      **((_BYTE **)v2 + 1) = 1;
      **(_BYTE **)(*((_DWORD *)v2 + 1) + 4) = 0;
      sub_179A0(*(_DWORD *)(*((_DWORD *)v2 + 1) + 4), a2);
    }
    else
    {
      *(_BYTE *)v4 = 1;
      *v6 = 1;
      **(_BYTE **)(*((_DWORD *)v2 + 1) + 4) = 0;
      v2 = *(_BYTE **)(*((_DWORD *)v2 + 1) + 4);
    }
  }
  result = (_BYTE *)*a2;
  *(_BYTE *)*a2 = 1;
  return result;
}

