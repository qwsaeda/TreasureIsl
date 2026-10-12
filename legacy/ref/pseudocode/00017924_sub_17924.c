/* ea:   0x00017924 */
/* name: sub_17924 */
/* size: 124 */
int __fastcall sub_17924(char *a1)
{
  char v2; // r0
  bool v3; // zf
  int v5; // r1
  int i; // r0
  int v7; // r2
  int v8; // r1

  v2 = *a1;
  v3 = v2 == 0;
  if ( v2 == 0 )
    v3 = *(_DWORD *)(*((_DWORD *)a1 + 1) + 4) == (_DWORD)a1;
  if ( v3 )
    return *((_DWORD *)a1 + 3);
  v5 = *((_DWORD *)a1 + 2);
  if ( v5 != 0 )
  {
    for ( i = *(_DWORD *)(v5 + 12); i != 0; i = *(_DWORD *)(i + 12) )
      v5 = i;
    return v5;
  }
  else
  {
    v7 = *((_DWORD *)a1 + 1);
    if ( a1 == *(char **)(v7 + 8) )
    {
      do
      {
        v8 = v7;
        v7 = *(_DWORD *)(v7 + 4);
      }
      while ( v8 == *(_DWORD *)(v7 + 8) );
    }
    return v7;
  }
}

