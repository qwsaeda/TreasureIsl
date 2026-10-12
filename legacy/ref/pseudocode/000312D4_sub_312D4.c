/* ea:   0x000312D4 */
/* name: sub_312D4 */
/* size: 260 */
int *__fastcall sub_312D4(int a1, _DWORD *a2)
{
  int *result; // r0
  int *v5; // r5
  int v6; // r0
  _WORD *v7; // r3
  _WORD *v8; // r3
  _WORD *v9; // r3
  int v10; // [sp+0h] [bp-1Ch] BYREF
  int v11; // [sp+4h] [bp-18h] BYREF
  int v12; // [sp+8h] [bp-14h] BYREF

  result = (int *)*(unsigned __int8 *)(a1 + 50);
  if ( (_BYTE)result != 0 )
  {
    v5 = *(int **)(a1 + 28);
    while ( 1 )
    {
      while ( 1 )
      {
        result = *(int **)(a1 + 32);
        if ( v5 == result )
          return result;
        v6 = *v5;
        if ( *v5 != 1 )
          break;
        v12 = *(unsigned __int8 *)(a1 + 50);
        v9 = (_WORD *)sub_2F550(a2, &v12);
        ++*v9;
LABEL_10:
        ++v5;
      }
      if ( v6 == 3 )
      {
        v11 = *(unsigned __int8 *)(a1 + 50);
        v8 = (_WORD *)(sub_2F550(a2, &v11) + 2);
        ++v5;
        *v8 += 50;
      }
      else
      {
        if ( v6 != 5 )
          goto LABEL_10;
        v10 = *(unsigned __int8 *)(a1 + 50);
        v7 = (_WORD *)(sub_2F550(a2, &v10) + 4);
        ++v5;
        *v7 += 20;
      }
    }
  }
  return result;
}

