/* ea:   0x00038778 */
/* name: sub_38778 */
/* size: 288 */
_DWORD *__fastcall sub_38778(_DWORD *a1, _DWORD *a2, int a3, int a4, unsigned __int8 *a5, int a6)
{
  _BYTE *v9; // r6
  int v10; // r2
  int v11; // r1
  int v12; // r0
  int *v13; // r1
  int v14; // r0
  int v15; // r1
  int v16; // t1

  if ( a4 == *a1 || a6 == 0 && (a3 != 0 || *a5 < (unsigned int)*(unsigned __int8 *)(a4 + 16)) )
  {
    v14 = sub_399E0(a1, a5);
    v9 = (_BYTE *)v14;
    *(_DWORD *)(a4 + 8) = v14;
    v15 = *a1;
    if ( a4 == *a1 )
    {
      *(_DWORD *)(v15 + 4) = v14;
      *(_DWORD *)(*a1 + 12) = v14;
      goto LABEL_13;
    }
    v16 = *(_DWORD *)(v15 + 8);
    v13 = (int *)(v15 + 8);
    v12 = v16;
  }
  else
  {
    v9 = sub_343AC((int)a1, 1);
    if ( v9 != (_BYTE *)-16 )
    {
      v9[16] = *a5;
      v10 = a5[2] | (a5[3] << 8);
      v11 = v10 | (a5[4] << 16) | (a5[5] << 24);
      *((_WORD *)v9 + 9) = v10;
      *((_WORD *)v9 + 10) = HIWORD(v11);
    }
    *(_DWORD *)(a4 + 12) = v9;
    v13 = (int *)(*a1 + 12);
    v12 = *v13;
  }
  if ( a4 == v12 )
    *v13 = (int)v9;
LABEL_13:
  *((_DWORD *)v9 + 1) = a4;
  *((_DWORD *)v9 + 2) = 0;
  *((_DWORD *)v9 + 3) = 0;
  sub_183FC(v9, (_DWORD *)(*a1 + 4));
  ++a1[1];
  *a2 = v9;
  return a2;
}

