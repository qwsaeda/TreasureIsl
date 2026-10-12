/* ea:   0x0001DDA4 */
/* name: sub_1DDA4 */
/* size: 76 */
int __fastcall sub_1DDA4(int a1, unsigned int a2)
{
  if ( a2 >= (*(_DWORD *)(a1 + 4) - *(_DWORD *)a1) >> 2 )
    return sub_1DF4C(a1, *(void **)(a1 + 4));
  else
    return sub_1DE28(a1, (void *)(*(_DWORD *)a1 + 4 * a2), *(void **)(a1 + 4));
}

