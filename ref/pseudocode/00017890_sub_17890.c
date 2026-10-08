/* ea:   0x00017890 */
/* name: sub_17890 */
/* size: 148 */
int __fastcall sub_17890(int a1, __int16 *a2, __int16 *a3)
{
  _WORD *v5; // r1
  __int16 v6; // t1

  v5 = *(_WORD **)a1;
  if ( a2 == a3 )
    goto LABEL_4;
  while ( v5 != *(_WORD **)(a1 + 4) )
  {
    v6 = *a2++;
    *v5++ = v6;
    if ( a2 == a3 )
      goto LABEL_4;
  }
  if ( a2 == a3 )
  {
LABEL_4:
    sub_13248(a1, v5, *(void **)(a1 + 4));
    return a1;
  }
  else
  {
    sub_18548(a1, a2);
    return a1;
  }
}

