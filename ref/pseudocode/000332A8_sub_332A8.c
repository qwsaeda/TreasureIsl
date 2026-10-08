/* ea:   0x000332A8 */
/* name: sub_332A8 */
/* size: 20 */
void __fastcall sub_332A8(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

