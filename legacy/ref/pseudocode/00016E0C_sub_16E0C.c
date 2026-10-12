/* ea:   0x00016E0C */
/* name: sub_16E0C */
/* size: 204 */
int __fastcall sub_16E0C(int a1, unsigned int a2)
{
  unsigned int v3; // r5
  _DWORD *v4; // r4
  _WORD *v5; // r3
  unsigned int v6; // r2

  v3 = a2;
  if ( sub_16D1C(a1) > a2 )
  {
    v4 = *(_DWORD **)(*(_DWORD *)a1 + 8);
    while ( v3 != 0 )
    {
      if ( *((_WORD *)v4 + 9) != 0 )
      {
        v5 = (_WORD *)v4 + 9;
        v6 = *((unsigned __int16 *)v4 + 9);
        if ( v6 > 1 )
          *v5 = v6 - 1;
        else
          *v5 = 0;
      }
      v4 = sub_16A30(v4);
      --v3;
      if ( v4 == *(_DWORD **)a1 )
        v4 = *(_DWORD **)(*(_DWORD *)a1 + 8);
    }
    return sub_16F5C(a1);
  }
  else
  {
    sub_16ED8(a1);
    sub_16F5C(a1);
    return sub_16F5C(a1);
  }
}

