/* ea:   0x00052CE8 */
/* name: sub_52CE8 */
/* size: 68 */
_DWORD **__fastcall sub_52CE8(int a1)
{
  _DWORD **v2; // r0
  _DWORD *v3; // r4
  void *v4; // r0
  _DWORD **result; // r0

  v2 = *(_DWORD ***)a1;
  v3 = *v2;
  if ( *v2 != v2 )
  {
    do
    {
      v4 = v3;
      v3 = (_DWORD *)*v3;
      free(v4);
    }
    while ( v3 != *(_DWORD **)a1 );
  }
  result = *(_DWORD ***)a1;
  **(_DWORD **)a1 = *(_DWORD *)a1;
  *(_DWORD *)(*(_DWORD *)a1 + 4) = *(_DWORD *)a1;
  return result;
}

