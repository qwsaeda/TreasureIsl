/* ea:   0x0003E42C */
/* name: sub_3E42C */
/* size: 244 */
int *__fastcall sub_3E42C(_DWORD *a1, int *a2, int a3, int a4, unsigned __int16 *a5, int a6)
{
  int v9; // r4
  int v10; // r0
  int *v11; // r1
  int v12; // r0
  int v13; // r1
  int v14; // t1

  if ( a4 == *a1 || a6 == 0 && (a3 != 0 || *a5 < (unsigned int)*(unsigned __int16 *)(a4 + 16)) )
  {
    v12 = sub_3EC10(a1, a5);
    v9 = v12;
    *(_DWORD *)(a4 + 8) = v12;
    v13 = *a1;
    if ( a4 == *a1 )
    {
      *(_DWORD *)(v13 + 4) = v12;
      *(_DWORD *)(*a1 + 12) = v12;
      goto LABEL_11;
    }
    v14 = *(_DWORD *)(v13 + 8);
    v11 = (int *)(v13 + 8);
    v10 = v14;
  }
  else
  {
    v9 = sub_3EC10(a1, a5);
    *(_DWORD *)(a4 + 12) = v9;
    v11 = (int *)(*a1 + 12);
    v10 = *v11;
  }
  if ( a4 == v10 )
    *v11 = v9;
LABEL_11:
  *(_DWORD *)(v9 + 4) = a4;
  *(_DWORD *)(v9 + 8) = 0;
  *(_DWORD *)(v9 + 12) = 0;
  sub_183FC((_BYTE *)v9, (_DWORD *)(*a1 + 4));
  ++a1[1];
  *a2 = v9;
  return a2;
}

