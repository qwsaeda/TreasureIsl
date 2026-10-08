/* ea:   0x00064B80 */
/* name: sub_64B80 */
/* size: 380 */
_DWORD *__fastcall sub_64B80(int a1, int a2, int a3, int *a4)
{
  int v4; // r4
  int v7; // r7
  unsigned int v8; // r6
  char *v9; // r5
  _BYTE *v11; // r8
  unsigned int v12; // r9
  char *v13; // r3
  char *v14; // r2
  int v15; // r5
  char *i; // r4
  char v17; // t1
  _BYTE *v18; // r8
  char v19; // t1
  char v20; // t1
  _DWORD *v21; // [sp+8h] [bp-34h]
  char *v23; // [sp+10h] [bp-2Ch]

  v4 = a2;
  if ( a2 == 0 || *a4 != 24 )
    __debugbreak();
  v21 = sub_62694(a4[1], a4[2], *a4, 0);
  if ( v21 != nullptr )
  {
    v7 = v4 + 128;
    v8 = *(unsigned __int16 *)(v4 + 66) * *(unsigned __int8 *)(v4 + 65);
    v9 = (char *)operator new(v8);
    v23 = v9;
    if ( v9 == nullptr )
    {
      free(v21);
      return nullptr;
    }
    v11 = (_BYTE *)v21[7];
    v12 = 0;
    if ( a4[2] == 0 )
    {
      operator delete(v9);
      return v21;
    }
    while ( 1 )
    {
      v7 = sub_64CFC(a1, v9, v8, v7, v4, a3);
      if ( v7 == 0 )
        break;
      v13 = v9;
      v14 = &v9[*(unsigned __int16 *)(v4 + 66)];
      v15 = a4[1];
      for ( i = &v14[*(unsigned __int16 *)(v4 + 66)]; v15 != 0; v11 = v18 + 1 )
      {
        v17 = *v13++;
        --v15;
        *v11 = v17;
        v18 = v11 + 1;
        v19 = *v14++;
        *v18++ = v19;
        v20 = *i++;
        *v18 = v20;
      }
      ++v12;
      v9 = v23;
      if ( v12 >= a4[2] )
      {
        operator delete(v23);
        return v21;
      }
      v4 = a2;
    }
    free(v21);
    operator delete(v9);
  }
  return nullptr;
}

