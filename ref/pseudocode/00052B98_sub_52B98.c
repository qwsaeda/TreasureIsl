/* ea:   0x00052B98 */
/* name: sub_52B98 */
/* size: 336 */
int *__fastcall sub_52B98(int *result, char *a2, size_t a3, unsigned __int8 *a4)
{
  int *v4; // r9
  char *v7; // r2
  int v8; // r5
  unsigned __int8 v9; // r6
  int v10; // r4
  char *v11; // r1
  signed int v12; // r4
  char *v13; // r2
  _BYTE *v14; // r3
  signed int i; // r1
  char v16; // t1
  char v17; // [sp+9h] [bp-1Fh] BYREF

  v4 = result;
  if ( a3 != 0 )
  {
    v7 = (char *)result[1];
    if ( result[2] - (int)v7 < a3 )
    {
      return (int *)sub_1631C(result, a2, a4, (int)&v17, a3, 0);
    }
    else
    {
      v8 = v7 - a2;
      v9 = *a4;
      v10 = result[1];
      if ( v7 - a2 <= a3 )
      {
        if ( a3 != v8 )
        {
          v11 = &a2[a3];
          do
            *v7++ = v9;
          while ( v7 != v11 );
        }
        v12 = v10 - (_DWORD)a2;
        v13 = a2;
        v14 = (_BYTE *)(result[1] - v8 + a3);
        result[1] = (int)v14;
        if ( v12 > 0 )
        {
          for ( i = v12; i != 0; --i )
          {
            v16 = *v13++;
            *v14++ = v16;
          }
        }
        result[1] += v8;
        return (int *)memset(a2, v9, v12);
      }
      else
      {
        sub_5360C(&v7[-a3], v7);
        v4[1] += a3;
        if ( (int)(v10 - (_DWORD)a2 - a3) > 0 )
          memmove(&a2[a3], a2, v10 - (_DWORD)a2 - a3);
        return (int *)memset(a2, v9, a3);
      }
    }
  }
  return result;
}

