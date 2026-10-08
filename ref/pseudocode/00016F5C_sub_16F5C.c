/* ea:   0x00016F5C */
/* name: sub_16F5C */
/* size: 96 */
BOOL __fastcall sub_16F5C(int a1)
{
  _DWORD *i; // r5
  BOOL result; // r0

  for ( i = *(_DWORD **)(*(_DWORD *)a1 + 8); ; i = sub_16A30(i) )
  {
    result = i != *(_DWORD **)a1;
    if ( i == *(_DWORD **)a1 )
      break;
    if ( *((_WORD *)i + 9) == 0 )
      sub_16FC4(a1, i, *((unsigned __int16 *)i + 9));
  }
  return result;
}

