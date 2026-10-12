/* ea:   0x0006179C */
/* name: sub_6179C */
/* size: 244 */
char **__fastcall sub_6179C(_DWORD *a1, char **a2, int a3, char *a4, _DWORD *a5, int a6)
{
  char *v9; // r5
  char *v10; // r0
  char **v11; // r1
  int v12; // r0
  int v13; // r1
  char *v14; // t1

  if ( a4 == (char *)*a1 || a6 == 0 && (a3 != 0 || *a5 < *((_DWORD *)a4 + 4)) )
  {
    v12 = sub_619A0(a1, a5);
    v9 = (char *)v12;
    *((_DWORD *)a4 + 2) = v12;
    v13 = *a1;
    if ( a4 == (char *)*a1 )
    {
      *(_DWORD *)(v13 + 4) = v12;
      *(_DWORD *)(*a1 + 12) = v12;
      goto LABEL_13;
    }
    v14 = *(char **)(v13 + 8);
    v11 = (char **)(v13 + 8);
    v10 = v14;
  }
  else
  {
    v9 = (char *)sub_19894((int)a1, 1);
    if ( v9 != (char *)-16 )
      sub_61F64(v9 + 16, a5);
    *((_DWORD *)a4 + 3) = v9;
    v11 = (char **)(*a1 + 12);
    v10 = *v11;
  }
  if ( a4 == v10 )
    *v11 = v9;
LABEL_13:
  *((_DWORD *)v9 + 1) = a4;
  *((_DWORD *)v9 + 2) = 0;
  *((_DWORD *)v9 + 3) = 0;
  sub_183FC(v9, (_DWORD *)(*a1 + 4));
  ++a1[1];
  *a2 = v9;
  return a2;
}

